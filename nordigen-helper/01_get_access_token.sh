#!/bin/bash

# https://nordigen.com/en/account_information_documenation/integration/quickstart_guide/

echo '
Got to https://nordigen.com and create a free
account. Now create some user secrets.
They will have an secret_id and secret_key

Also install jq (JSON formatting in bash)

'

echo Enter the secret_id:
echo -n "> "
read SECRET_ID

echo Enter the secret_key:
echo -n "> "
read SECRET_KEY


RES=$(curl -X POST "https://ob.nordigen.com/api/v2/token/new/" \
  -H "accept: application/json" \
  -H "Content-Type: application/json" \
  -d '{"secret_id": "'$SECRET_ID'", "secret_key": "'$SECRET_KEY'"}')

echo Result plain:
echo $RES | jq
echo

echo Access token:
echo $RES | jq -r .access
echo $RES | jq -r .access > .accesstoken
echo The access token has been written to .accesstoken
echo

echo Refresh token:
echo $RES | jq -r .refresh
echo $RES | jq -r .refresh > .refreshtoken
echo The refresh token has been written to .refreshtoken

