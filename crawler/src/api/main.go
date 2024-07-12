package main

import (
	"database/sql"
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"os"
	"sort"
	"strings"

	"github.com/gorilla/mux"
	"github.com/lib/pq"
	"github.com/reiver/go-porterstemmer"
)

type Page struct {
	ID          int
	URL         string
	Title       string
	Description string
}

type Keyword struct {
	Word      string
	Frequency int
}

type CompletePage struct {
	Page     Page
	Keywords []Keyword
}

type SearchEngine struct {
	db *sql.DB
}

func NewSearchEngine(connStr string) (*SearchEngine, error) {
	db, err := sql.Open("postgres", connStr)
	if err != nil {
		return nil, err
	}

	return &SearchEngine{db: db}, nil
}

func (se *SearchEngine) search(query string) ([]CompletePage, error) {
	if query == "" {
		return nil, fmt.Errorf("no query provided")
	}

	// Extract keywords from query
	keywords := extractKeywords(query)

	// Get pages matching the keywords
	pages, err := se.getPagesWithKeywords(keywords)
	if err != nil {
		return nil, err
	}

	// Map the pages to their keywords
	unorderedPages := []CompletePage{}
	for _, page := range pages {
		pageID := page.ID
		keywords, err := se.getKeywordsByPageID(pageID)
		if err != nil {
			return nil, err
		}
		unorderedPages = append(unorderedPages, CompletePage{Page: page, Keywords: keywords})
	}

	// Find the backlinks for each page
	backlinks := make(map[int]int)
	for _, page := range unorderedPages {
		pageBacklinks, err := se.getBacklinks(page.Page.ID)
		if err != nil {
			return nil, err
		}
		for _, backlink := range pageBacklinks {
			backlinks[backlink]++
		}
	}

	// Calculate relevance scores
	relevanceScores := make(map[int]int)
	for _, page := range unorderedPages {
		score := 0
		for _, keyword := range page.Keywords {
			if freq, found := keywords[keyword.Word]; found {
				score += freq * keyword.Frequency
			}
		}
		relevanceScores[page.Page.ID] = score
	}

	// Calculate page ranks
	ratingFactor := 1.0
	rankerConstant := 0.85
	pageRanks := make(map[int]float64)
	for _, page := range unorderedPages {
		rank := ratingFactor
		for backlinkID, backlinkCount := range backlinks {
			if backlinkID == page.Page.ID {
				continue
			}
			rank += float64(relevanceScores[backlinkID]) / float64(backlinkCount)
		}
		rank *= rankerConstant
		pageRanks[page.Page.ID] = rank
	}

	// Sort pages by rank
	sort.Slice(unorderedPages, func(i, j int) bool {
		return pageRanks[unorderedPages[i].Page.ID] > pageRanks[unorderedPages[j].Page.ID]
	})

	return unorderedPages, nil
}

func extractKeywords(query string) map[string]int {
	words := strings.Fields(query)
	stemmedWords := make(map[string]int)
	for _, word := range words {
		stemmedWord := porterstemmer.StemString(strings.ToLower(word))
		stemmedWords[stemmedWord]++
	}
	return stemmedWords
}

func (se *SearchEngine) getPagesWithKeywords(keywords map[string]int) ([]Page, error) {
	// Construct the SQL query to fetch pages containing the keywords
	keywordList := make([]string, 0, len(keywords))
	for keyword := range keywords {
		keywordList = append(keywordList, keyword)
	}

	query := `
		SELECT DISTINCT p.id, p.url, p.title, p.description
		FROM pages p
		JOIN page_keywords pk ON p.id = pk.page_id
		WHERE pk.word = ANY($1)
	`
	rows, err := se.db.Query(query, pq.Array(keywordList))
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var pages []Page
	for rows.Next() {
		var page Page
		if err := rows.Scan(&page.ID, &page.URL, &page.Title, &page.Description); err != nil {
			return nil, err
		}
		pages = append(pages, page)
	}
	return pages, nil
}

func (se *SearchEngine) getKeywordsByPageID(pageID int) ([]Keyword, error) {
	query := `
		SELECT word, frequency
		FROM page_keywords
		WHERE page_id = $1
	`
	rows, err := se.db.Query(query, pageID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var keywords []Keyword
	for rows.Next() {
		var keyword Keyword
		if err := rows.Scan(&keyword.Word, &keyword.Frequency); err != nil {
			return nil, err
		}
		keywords = append(keywords, keyword)
	}
	return keywords, nil
}

func (se *SearchEngine) getBacklinks(pageID int) ([]int, error) {
	query := `
		SELECT source_page_id
		FROM backlinks
		WHERE target_page_id = $1
	`
	rows, err := se.db.Query(query, pageID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var backlinks []int
	for rows.Next() {
		var backlinkID int
		if err := rows.Scan(&backlinkID); err != nil {
			return nil, err
		}
		backlinks = append(backlinks, backlinkID)
	}
	return backlinks, nil
}

func searchHandler(se *SearchEngine) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		query := r.URL.Query().Get("q")
		if query == "" {
			http.Error(w, "No query provided", http.StatusBadRequest)
			return
		}

		results, err := se.search(query)
		if err != nil {
			http.Error(w, err.Error(), http.StatusInternalServerError)
			return
		}

		w.Header().Set("Content-Type", "application/json")
		if err := json.NewEncoder(w).Encode(results); err != nil {
			http.Error(w, err.Error(), http.StatusInternalServerError)
		}
	}
}

func main() {
	connStr := os.Getenv("PSOTGRES_CONN")
	se, err := NewSearchEngine(connStr)
	if err != nil {
		log.Fatalf("Failed to initialize search engine: %v", err)
	}

	r := mux.NewRouter()
	r.HandleFunc("/search", searchHandler(se)).Methods("GET")

	http.Handle("/", r)
	log.Println("Server started on :8080")
	log.Fatal(http.ListenAndServe(":8080", nil))
}
