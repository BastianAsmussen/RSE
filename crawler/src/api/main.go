package main

import (
	"database/sql"
	"fmt"
	"net/http"
	"os"

	_ "github.com/lib/pq"
)

type SearchEngine struct {
	pgConn *sql.DB
}

type Page struct {
	ID          int
	URL         string
	Title       string
	Description string
}

func NewSearchEngine(pgConnStr string) *SearchEngine {
	conn, err := sql.Open("postgres", pgConnStr)
	if err != nil {
		fmt.Printf("Unable to connect to database: %v\n", err)
		panic(err)
	}
	return &SearchEngine{pgConn: conn}
}

func (se *SearchEngine) search(query string) ([]Page, error) {
	// Simple keyword extraction for search query
	words := strings.Fields(query)

	var pages []Page

	query = `
		SELECT id, url, title, description 
		FROM pages 
		WHERE title ILIKE $1 OR description ILIKE $2
	`
	rows, err := se.pgConn.Query(query, "%"+query+"%", "%"+query+"%")
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	for rows.Next() {
		var page Page
		if err := rows.Scan(&page.ID, &page.URL, &page.Title, &page.Description); err != nil {
			return nil, err
		}
		pages = append(pages, page)
	}

	return pages, nil
}

func searchHandler(se *SearchEngine) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		query := r.URL.Query().Get("q")
		if query == "" {
			http.Error(w, "No query provided", http.StatusBadRequest)
			return
		}

		pages, err := se.search(query)
		if err != nil {
			http.Error(w, "Error searching pages", http.StatusInternalServerError)
			return
		}

		for _, page := range pages {
			fmt.Fprintf(w, "URL: %s\nTitle: %s\nDescription: %s\n\n", page.URL, page.Title, page.Description)
		}
	}
}

func main() {
	pgConnStr := os.Getenv("POSTGRES_CONN")
	se := NewSearchEngine(pgConnStr)

	http.HandleFunc("/search", searchHandler(se))

	fmt.Println("Search engine running on port 8080")
	if err := http.ListenAndServe(":8080", nil); err != nil {
		fmt.Printf("Error starting server: %v\n", err)
	}
}
