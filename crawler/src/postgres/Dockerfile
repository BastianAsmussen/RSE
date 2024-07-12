FROM postgres:alpine

COPY init_db.sql /docker-entrypoint-initdb.d/

ENV POSTGRES_DB=gse
ENV POSTGRES_USER=postgres
ENV POSTGRES_PASSWORD=toor

EXPOSE 5432

