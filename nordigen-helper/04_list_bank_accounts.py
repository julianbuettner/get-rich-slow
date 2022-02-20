#!/usr/bin/env python3

from requests import post, get
from os import listdir

ACCOUNTS_URL = "https://ob.nordigen.com/api/v2/requisitions/{}"
BALANCES_URL = "https://ob.nordigen.com/api/v2/accounts/{}/balances/"


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


def get_requistions() -> list:
  result = []
  for entry in listdir("."):
    if not entry.startswith(".requistion"):
      continue
    result.append(open(entry, "r").read())
  return result


def print_account_balances(accesstoken, requisition):
  accounts_res = post(
    ACCOUNTS_URL.format(requisition),
    headers={
      "Authorization": "Bearer {}".format(accesstoken)
    }
  )
  institution_id = accounts_res.json()["institution_id"]
  accounts = accounts_res.json()["accounts"]

  for account in accounts:
    balance_res = get(
      BALANCES_URL.format(account),
      headers={
        "Authorization": "Bearer {}".format(accesstoken)
      }
    )
    balance = balance_res.json()["balances"][0]["balanceAmount"]["amount"]
    currency = balance_res.json()["balances"][0]["balanceAmount"]["currency"]

    print("Account ID {} (Balance {} {})".format(account, balance, currency))

def main():
  accesstoken = get_accesstoken()
  requisitions = get_requistions()
  for req in requisitions:
    print_account_balances(accesstoken, req)

  print()
  print("For the configuration ~/.get-rich-slow.yaml, you also need")
  print("The refresh token:\n\t{}".format(get_refreshtoken()))

main()
