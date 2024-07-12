FROM redis:alpine

COPY scripts/seed_urls.txt /data/seed_urls.txt
COPY scripts/seed_urls.sh /data/seed_urls.sh

RUN chmod +x /data/seed_urls.sh

CMD ["redis-server", "--appendonly", "yes", "--dir", "/data", "--dbfilename", "dump.rdb"]
