#!/bin/sh
set -u
set -x
set -e

# Don't include trailing slash
URL_BASE="http://localhost:8080"
CURL_ARGS=""

curl $CURL_ARGS "${URL_BASE}/index.html"
curl $CURL_ARGS "${URL_BASE}/new"
curl $CURL_ARGS -X POST \
    -d "title=test&due-date=2021-07-28&start-date=2021-07-26" \
    "${URL_BASE}/new"
TODO1=$(curl $CURL_ARGS "${URL_BASE}/index.html" | grep id= | head -n 1 | sed -n 's/^.*id="\(.*\)".*$/\1/p')
curl "${URL_BASE}/new"
curl $CURL_ARGS -X POST \
    -d "title=test2&due-date=2021-07-31&start-date=2021-07-28" \
    "${URL_BASE}/new"
curl $CURL_ARGS "${URL_BASE}/index.html"
curl $CURL_ARGS -X POST \
    -d "title=dingbats&due-date=2021-07-30&start-date=2021-07-26" \
    "${URL_BASE}/edit/?id=${TODO1}"
curl $CURL_ARGS "${URL_BASE}/index.html"
curl $CURL_ARGS -X POST "${URL_BASE}/complete/?id=${TODO1}"
