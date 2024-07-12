#!/bin/sh

cat /data/seed_urls.txt | while read url; do
  echo "Pushing seed URL: $url"
  redis-cli -h localhost -p 6379 LPUSH url_queue "$url"
done

