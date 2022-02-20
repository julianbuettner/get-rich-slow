#!/usr/bin/env python3

from requests import post

LINK_URL = "https://ob.nordigen.com/api/v2/requisitions/"

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

def get_bank_id() -> str:
  print("Enter the Id of your desired bank:")
  return input("> ")


def get_link(accesstoken: str, bank_id: str) -> str:
  res = post(
    LINK_URL,
    json={
      "redirect": "https://github.com/julianbuettner/get-rich-slow",
      "institution_id": bank_id,
    },
    headers={
      "Authorization": "Bearer {}".format(accesstoken)
    }
  )
  requistion_file = ".requistion-id-{}.txt".format(bank_id).lower()
  with open(requistion_file, "w") as f:
    f.write(res.json()["id"])

  print(
    "The requisition ID for your bank has already been written into\n"
    "\t{}}n"
    "and only need your activation now."
    .format(requistion_file)
  )

  return res.json()["link"]


def main():
  accesstoken = get_accesstoken()
  bank_id = get_bank_id()

  link = get_link(accesstoken, bank_id)
  print("Now visit:")
  print("\t{}".format(link))

main()
