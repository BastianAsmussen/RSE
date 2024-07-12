package crawler

import (
	"bytes"
	"context"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"os"
	"strings"
	"sync"
	"time"

	"github.com/go-redis/redis/v8"
	"github.com/jackc/pgx/v4"
	"golang.org/x/net/html"
)

type Crawler struct {
	redisClient *redis.Client
	ctx         context.Context
	mu          sync.Mutex
	wg          sync.WaitGroup
	client      *http.Client
	pgConn      *pgx.Conn
}

func NewCrawler(redisAddr string, pgConnStr string) *Crawler {
	ctx := context.Background()
	rdb := redis.NewClient(&redis.Options{
		Addr: redisAddr,
	})

	conn, err := pgx.Connect(ctx, pgConnStr)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Unable to connect to database: %v\n", err)
		os.Exit(1)
	}

	return &Crawler{
		redisClient: rdb,
		ctx:         ctx,
		client: &http.Client{
			Timeout: 10 * time.Second,
		},
		pgConn: conn,
	}
}

func (c *Crawler) StartCrawling() {
	for {
		rawurl, err := c.redisClient.RPop(c.ctx, "url_queue").Result()
		if err == redis.Nil {
			fmt.Println("No URLs to crawl.")
			time.Sleep(5 * time.Second)
			continue
		} else if err != nil {
			fmt.Println("Error fetching URL from queue:", err)
			continue
		}

		c.wg.Add(1)
		go c.fetch(rawurl, 0)
	}
}

func (c *Crawler) fetch(rawurl string, retryCount int) {
	defer c.wg.Done()

	// Parse URL
	parsedURL, err := url.Parse(rawurl)
	if err != nil {
		fmt.Println("Invalid URL:", rawurl)
		return
	}

	// Check if URL has been visited
	visited, err := c.redisClient.SIsMember(c.ctx, "visited_urls", parsedURL.String()).Result()
	if err != nil {
		fmt.Println("Error checking visited URLs:", err)
		return
	}
	if visited {
		return
	}

	// Make HTTP request
	resp, err := c.client.Get(parsedURL.String())
	if err != nil {
		fmt.Println("Error fetching URL:", rawurl, err)
		if retryCount < 3 {
			fmt.Println("Retrying URL:", rawurl)
			time.Sleep(2 * time.Second)
			c.wg.Add(1)
			go c.fetch(rawurl, retryCount+1)
		}
		return
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		fmt.Println("Non-OK HTTP status:", resp.StatusCode, "for URL:", rawurl)
		return
	}

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		fmt.Println("Error reading response body:", err)
		return
	}

	// Extract title and description
	title, description := extractMetadata(body)

	// Save website content to PostgreSQL
	c.saveContent(parsedURL.String(), title, description)

	// Mark URL as visited
	err = c.redisClient.SAdd(c.ctx, "visited_urls", parsedURL.String()).Err()
	if err != nil {
		fmt.Println("Error adding URL to visited set:", err)
	}

	// Extract and enqueue links
	links := extractLinks(parsedURL, body)
	for _, link := range links {
		err = c.redisClient.LPush(c.ctx, "url_queue", link).Err()
		if err != nil {
			fmt.Println("Error pushing URL to queue:", err)
		}
	}
}

func (c *Crawler) saveContent(url, title, description string) {
	query := `
		INSERT INTO pages (url, title, description)
		VALUES ($1, $2, $3)
		ON CONFLICT (url) 
		DO UPDATE SET last_crawled_at = NOW(), title = $2, description = $3
	`

	_, err := c.pgConn.Exec(c.ctx, query, url, title, description)
	if err != nil {
		fmt.Println("Error inserting data into PostgreSQL:", err)
	}
}

func extractMetadata(body []byte) (title, description string) {
	doc, err := html.Parse(bytes.NewReader(body))
	if err != nil {
		fmt.Println("Error parsing HTML:", err)
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
		fmt.Println("Error parsing HTML:", err)
		return links
	}

	var f func(*html.Node)
	f = func(n *html.Node) {
		if n.Type == html.ElementNode && n.Data == "a" {
			for _, a := range n.Attr {
				if a.Key == "href" {
					link := a.Val

					// Resolve relative URLs
					resolvedURL := resolveURL(baseURL, link)
					if resolvedURL != "" && isValidURL(resolvedURL) {
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

	// Check if the scheme is HTTP or HTTPS
	if parsedURL.Scheme != "http" && parsedURL.Scheme != "https" {
		return false
	}

	// Ignore URLs with fragment identifiers
	if parsedURL.Fragment != "" {
		return false
	}

	return true
}
