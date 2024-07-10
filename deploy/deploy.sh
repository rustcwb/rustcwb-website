#!/bin/bash
cd /app
tar -xzf public.tar.gz
tar -xzf binary.tar.gz
rm *.tar.gz
killall web-server
nohup ./web-server > nohup.out 2>&1 &
exit 0