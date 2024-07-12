package main

import (
	"os"

	"asmussen.tech/crawler/crawler"
)

func main() {
	redisAddr := os.Getenv("REDIS_ADDR")
	pgConnStr := os.Getenv("POSTGRES_CONN")

	c := crawler.NewCrawler(redisAddr, pgConnStr)
	c.StartCrawling()
}
