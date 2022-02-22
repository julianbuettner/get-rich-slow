#!/usr/bin/env python3

from requests import get


REQUISITION_URL = "https://ob.nordigen.com/api/v2/requisitions/"

def _get_token(tok) -> str:
  try:
    with open(tok, "r") as f:
      result = f.read().strip()
  except FileNotFoundError:
    print("File {} does not exist".format(tok))
    print("Run ./01_get_accesstoken.sh")
    quit()
  if not result:
    print("File {} is empty".format(tok))
    print("Run ./01_get_accesstoken.sh")
    quit()
  return result


def get_accesstoken():
  return _get_token(".accesstoken")


def get_refreshtoken():
  return _get_token(".refreshtoken")


def print_requisitions(accesstoken):
  requisition_res = get(
    REQUISITION_URL,
    headers={
      "Authorization": "Bearer {}".format(accesstoken)
    }
  )

  for req in requisition_res.json()["results"]:
    print(
      "Accounts ({}) {} {}:"
      .format(len(req["accounts"]), req["institution_id"], req["id"])
    )
    for acc in req["accounts"]:
      print("\t{}".format(acc))

def main():
  access = get_accesstoken()
  print_requisitions(access)

main()
