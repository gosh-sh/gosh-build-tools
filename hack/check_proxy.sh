#!/bin/bash

PROXY_IP=localhost

PROXY_IP=${PROXY_IP:-127.0.0.1}
export http_proxy=http://"$PROXY_IP":8000/
export https_proxy=http://"$PROXY_IP":8000/

URL='git.gosh.sh/test/test2.txt'

wget -q --spider $URL
# if wget -q --spider $URL; then
#     STATUS="Proxy is working."
# else
#     STATUS="Proxy isn't working"
# fi
# echo "$STATUS"
