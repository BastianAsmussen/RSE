package main

import (
	"log"
	"os"

	"crawler"
)

func main() {
	redisAddr := os.Getenv("REDIS_ADDR")
	pgConnStr := os.Getenv("POSTGRES_CONN")

	c := crawler.NewCrawler(redisAddr, pgConnStr)
	c.StartCrawling()
}

