package crawler

import (
	"bytes"
	"context"
	"fmt"
	"io"
	"math/rand"
	"net/http"
	"net/url"
	"os"
	"strings"
	"sync"
	"time"

	"github.com/go-redis/redis/v8"
	"github.com/jackc/pgx/v4"
	"github.com/jackc/pgx/v4/pgxpool"
	"github.com/temoto/robotstxt"
	"golang.org/x/net/html"
)

const (
	RequestTimeout     = 20 * time.Second
	RetryDelay         = 2 * time.Second
	MaxRetries         = 3
	URLQueue           = "url_queue"
	VisitedURLsSet     = "visited_urls"
	UserAgent          = "GSE-Bot"
	RevisitDelay       = 10 * time.Minute
)

var SeedURLs = []string{
	"https://www.google.com/",
	"https://www.bing.com/",
	"https://www.yahoo.com/",
	"https://www.bbc.com/",
	"https://www.nytimes.com/",
	"https://www.cnn.com/",
	"https://www.aljazeera.com/",
	"https://www.facebook.com/",
	"https://www.twitter.com/",
	"https://www.instagram.com/",
	"https://www.linkedin.com/",
	"https://www.ncbi.nlm.nih.gov/",
	"https://www.academia.edu/",
	"https://scholar.google.com/",
	"https://www.amazon.com/",
	"https://www.ebay.com/",
	"https://www.walmart.com/",
	"https://www.usa.gov/",
	"https://www.gov.uk/",
	"https://www.australia.gov.au/",
	"https://www.wordpress.com/",
	"https://www.blogger.com/",
	"https://medium.com/",
	"https://www.wikipedia.org/",
	"https://www.britannica.com/",
	"https://www.dictionary.com/",
	"https://www.techcrunch.com/",
	"https://www.stackoverflow.com/",
	"https://www.reddit.com/r/technology/",
	"https://www.harvard.edu/",
	"https://www.mit.edu/",
	"https://www.ox.ac.uk/",
	"https://www.data.gov/",
	"https://www.worldbank.org/",
	"https://www.datahub.io/",
	"https://www.youtube.com/",
	"https://www.vimeo.com/",
	"https://www.dailymotion.com/",
	"https://www.reddit.com/",
	"https://www.quora.com/",
	"https://www.stackexchange.com/",
	"https://www.webmd.com/",
	"https://www.mayoclinic.org/",
	"https://www.cdc.gov/",
}

type Crawler struct {
	redisClient *redis.Client
	ctx         context.Context
	mu          sync.Mutex
	wg          sync.WaitGroup
	client      *http.Client
	pgPool      *pgxpool.Pool
}

func NewCrawler(redisAddr string, pgConnStr string) *Crawler {
	ctx := context.Background()
	rdb := redis.NewClient(&redis.Options{
		Addr: redisAddr,
	})

	pool, err := pgxpool.Connect(ctx, pgConnStr)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Unable to connect to database: %v\n", err)
		os.Exit(1)
	}

	return &Crawler{
		redisClient: rdb,
		ctx:         ctx,
		client: &http.Client{
			Timeout: RequestTimeout,
		},
		pgPool: pool,
	}
}

func (c *Crawler) StartCrawling() {
	for {
		rawurl, err := c.redisClient.RPop(c.ctx, URLQueue).Result()
		if err == redis.Nil {
			fmt.Println("No URLs to crawl, selecting a seed URL...")
			rawurl = SeedURLs[rand.Intn(len(SeedURLs))]
		} else if err != nil {
			fmt.Println("Error fetching URL from queue:", err)
			time.Sleep(5 * time.Second)
			continue
		}

		c.wg.Add(1)
		go c.fetch(rawurl, 0)
	}
}

func (c *Crawler) fetch(rawurl string, retryCount int) {
	defer c.wg.Done()
	fmt.Printf("Fetching URL: %s (Retry count: %d)\n", rawurl, retryCount)

	parsedURL, err := url.Parse(rawurl)
	if err != nil {
		fmt.Printf("Invalid URL: %s, Error: %v\n", rawurl, err)
		return
	}

	if !c.shouldVisit(parsedURL.String()) {
		fmt.Printf("URL visited recently: %s, re-queueing\n", parsedURL.String())
		err = c.redisClient.LPush(c.ctx, URLQueue, parsedURL.String()).Err()
		if err != nil {
			fmt.Printf("Error pushing URL to queue: %v\n", err)
		}
		return
	}

	visited, err := c.redisClient.SIsMember(c.ctx, VisitedURLsSet, parsedURL.String()).Result()
	if err != nil {
		fmt.Printf("Error checking visited URLs: %v\n", err)
		return
	}
	if visited {
		fmt.Printf("URL already visited: %s\n", parsedURL.String())
		return
	}

	if !c.isAllowedByRobots(parsedURL) {
		fmt.Printf("URL disallowed by robots.txt: %s\n", parsedURL.String())
		return
	}

	ctx, cancel := context.WithTimeout(c.ctx, RequestTimeout)
	defer cancel()

	req, err := http.NewRequestWithContext(ctx, "GET", parsedURL.String(), nil)
	if err != nil {
		fmt.Printf("Error creating HTTP request: %v\n", err)
		return
	}

	resp, err := c.client.Do(req)
	if err != nil {
		fmt.Printf("Error fetching URL: %s, Error: %v\n", rawurl, err)
		if retryCount < MaxRetries {
			fmt.Printf("Retrying URL: %s\n", rawurl)
			time.Sleep(RetryDelay)
			c.wg.Add(1)
			go c.fetch(rawurl, retryCount+1)
		} else {
			fmt.Printf("Failed to fetch URL after %d attempts, selecting a new seed URL\n", MaxRetries)
			c.wg.Add(1)
			go c.fetch(SeedURLs[rand.Intn(len(SeedURLs))], 0)
		}
		return
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		fmt.Printf("Non-OK HTTP status: %d for URL: %s\n", resp.StatusCode, rawurl)
		return
	}

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		fmt.Printf("Error reading response body: %v\n", err)
		return
	}

	title, description := extractMetadata(body)
	c.saveContent(parsedURL.String(), title, description)

	err = c.redisClient.SAdd(c.ctx, VisitedURLsSet, parsedURL.String()).Err()
	if err != nil {
		fmt.Printf("Error adding URL to visited set: %v\n", err)
	}

	links := extractLinks(parsedURL, body)
	for _, link := range links {
		err = c.redisClient.LPush(c.ctx, URLQueue, link).Err()
		if err != nil {
			fmt.Printf("Error pushing URL to queue: %v\n", err)
		}
	}
}

func (c *Crawler) saveContent(url, title, description string) {
	query := `
		INSERT INTO pages (url, title, description, last_visited)
		VALUES ($1, $2, $3, NOW())
		ON CONFLICT (url) 
		DO UPDATE SET last_crawled_at = NOW(), title = $2, description = $3
	`

	_, err := c.pgPool.Exec(c.ctx, query, url, title, description)
	if err != nil {
		fmt.Printf("Error inserting data into PostgreSQL: %v\n", err)
	}
}

func (c *Crawler) shouldVisit(url string) bool {
	var lastVisited time.Time
	query := `SELECT last_visited FROM pages WHERE url = $1`
	err := c.pgPool.QueryRow(c.ctx, query, url).Scan(&lastVisited)
	if err != nil && err != pgx.ErrNoRows {
		fmt.Printf("Error querying last visited time: %v\n", err)
		return true
	}

	if err == pgx.ErrNoRows || time.Since(lastVisited) > RevisitDelay {
		return true
	}

	return false
}

func extractMetadata(body []byte) (title, description string) {
	doc, err := html.Parse(bytes.NewReader(body))
	if err != nil {
		fmt.Printf("Error parsing HTML: %v\n", err)
		return "", ""
	}

	var f func(*html.Node)
	f = func(n *html.Node) {
		if n.Type == html.ElementNode {
			if n.Data == "title" && title == "" {
				title = getTextContent(n)
			}
			if n.Data == "meta" {
				for _, a := range n.Attr {
					if a.Key == "name" && strings.ToLower(a.Val) == "description" {
						for _, a := range n.Attr {
							if a.Key == "content" {
								description = a.Val
								return
							}
						}
					}
				}
			}
		}
		for c := n.FirstChild; c != nil; c = c.NextSibling {
			f(c)
		}
	}
	f(doc)

	return
}

func getTextContent(n *html.Node) string {
	var buf bytes.Buffer
	for c := n.FirstChild; c != nil; c = c.NextSibling {
		if c.Type == html.TextNode {
			buf.WriteString(c.Data)
		}
		if c.FirstChild != nil {
			buf.WriteString(getTextContent(c))
		}
	}
	return buf.String()
}

func extractLinks(baseURL *url.URL, body []byte) []string {
	links := []string{}
	doc, err := html.Parse(bytes.NewReader(body))
	if err != nil {
		fmt.Printf("Error parsing HTML: %v\n", err)
		return links
	}

	var f func(*html.Node)
	f = func(n *html.Node) {
		if n.Type == html.ElementNode && n.Data == "a" {
			for _, a := range n.Attr {
				if a.Key == "href" {
					link := a.Val
					resolvedURL := resolveURL(baseURL, link)
					if resolvedURL != "" && isValidURL(resolvedURL) {
						fmt.Printf("Resolved URL: %s\n", resolvedURL)
						links = append(links, resolvedURL)
					}
					break
				}
			}
		}
		for c := n.FirstChild; c != nil; c = c.NextSibling {
			f(c)
		}
	}
	f(doc)

	return links
}

func resolveURL(base *url.URL, href string) string {
	u, err := url.Parse(href)
	if err != nil {
		return ""
	}
	return base.ResolveReference(u).String()
}

func isValidURL(link string) bool {
	parsedURL, err := url.Parse(link)
	if err != nil {
		return false
	}

	if parsedURL.Scheme != "http" && parsedURL.Scheme != "https" {
		return false
	}

	if parsedURL.Fragment != "" {
		return false
	}

	return true
}

func (c *Crawler) isAllowedByRobots(parsedURL *url.URL) bool {
	robotsURL := parsedURL.Scheme + "://" + parsedURL.Host + "/robots.txt"
	resp, err := c.client.Get(robotsURL)
	if err != nil {
		fmt.Printf("Error fetching robots.txt: %v\n", err)
		return true
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		fmt.Printf("Non-OK HTTP status for robots.txt: %d\n", resp.StatusCode)
		return true
	}

	data, err := io.ReadAll(resp.Body)
	if err != nil {
		fmt.Printf("Error reading robots.txt body: %v\n", err)
		return true
	}

	robots, err := robotstxt.FromBytes(data)
	if err != nil {
		fmt.Printf("Error parsing robots.txt: %v\n", err)
		return true
	}

	group := robots.FindGroup(UserAgent)
	if group == nil {
		group = robots.FindGroup("*")
	}

	return group.Test(parsedURL.Path)
}

