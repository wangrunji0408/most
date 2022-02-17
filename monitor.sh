#!/bin/sh

while true
do
ps -ef | grep "./target/release/most" | grep -v "grep"
if [ "$?" -eq 1 ]
then
nohup nice -n -20 ./target/release/most >> nohup.out &
echo "restart"
fi
sleep 5
done
