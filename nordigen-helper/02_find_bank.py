#!/usr/bin/env python3

from requests import get

BANK_LIST_URL = "https://ob.nordigen.com/api/v2/institutions/?country={cc}"

def get_accesstoken() -> str:
  try:
    with open(".accesstoken", "r") as f:
      result = f.read().strip()
  except FileNotFoundError:
    print("File .accesstoken does not exist")
    print("Run ./01_get_accesstoken.sh")
    quit()
  if not result:
    print("File .accesstoken is empty")
    print("Run ./01_get_accesstoken.sh")
    quit()
  return result

def get_country_code() -> str:
  print("Enter a country code for your bank(s):")
  return input("> ")

def get_bank_list(accesstoken: str, country_code: str):
  res = get(
    BANK_LIST_URL.format(cc=country_code),
    headers={
      "Authorization": "Bearer {}".format(accesstoken)
    }
  )
  return res.json()

def write_to_file(country_code: str, banks):
  with open("bank-list-{}.txt".format(country_code), "w") as f:
    for bank in banks:
      line = "{bic} {name} ID: {id}\n".format(
        bic=bank["bic"],
        name=bank["name"],
        id=bank["id"],
      )
      f.write(line)

def main():
  accesstoken = get_accesstoken()
  country_code = get_country_code()
  banks = get_bank_list(accesstoken, country_code)
  write_to_file(country_code, banks)
  print("Done")

  print(
    """
    Okay. All banks have been written to
      bank-list-{cc}.txt
    To find your bank run:
      cat bank-list-{cc}.txt | grep MYBIC
    of
      cat bank-list-{cc}.txt | grep "My Bank name"
    to find your bank. 
    """
    .format(cc=country_code)
  )

main()
