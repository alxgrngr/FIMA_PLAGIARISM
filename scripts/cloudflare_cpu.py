#!/usr/bin/env python3
"""Read staging CPU metrics without emitting credentials or participant data."""
import argparse
import datetime
import json
import os
import urllib.request

QUERY = """
query($account:String!, $since:Time!) {
  viewer { accounts(filter:{accountTag:$account}) {
    workersInvocationsAdaptive(limit:1000, filter:{
      datetime_geq:$since, scriptName:"nusa-receipts-staging"
    }) {
      sum { requests errors }
      quantiles { cpuTimeP50 cpuTimeP99 }
      dimensions { datetime httpResponseStatus status }
    }
  } }
}
"""


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--since", required=True, help="UTC ISO timestamp")
    args = parser.parse_args()
    since = datetime.datetime.fromisoformat(args.since.replace("Z", "+00:00"))
    if since.tzinfo is None:
        parser.error("--since must include a timezone")
    since = since.astimezone(datetime.timezone.utc).isoformat()
    request = urllib.request.Request(
        "https://api.cloudflare.com/client/v4/graphql",
        data=json.dumps({"query": QUERY, "variables": {
            "account": os.environ["CLOUDFLARE_ACCOUNT_ID"], "since": since,
        }}).encode(),
        headers={"Authorization": "Bearer " + os.environ["CLOUDFLARE_API_TOKEN"],
                 "Content-Type": "application/json"},
    )
    with urllib.request.urlopen(request, timeout=30) as response:
        result = json.load(response)
    if result.get("errors"):
        raise SystemExit("Cloudflare rejected the metrics query; no CPU result recorded.")
    rows = result["data"]["viewer"]["accounts"][0]["workersInvocationsAdaptive"]
    print(json.dumps({
        "worker": "nusa-receipts-staging", "since": since,
        "cpu_unit": "microseconds", "groups": rows,
        "limitations": [
            "Adaptive analytics; groups are not guaranteed to represent individual requests.",
            "No URL paths: HTTP 201 identifies receipt issuance in this Worker version.",
            "Results may lag; up to 1000 groups. Not a load test or automatic deployment approval.",
        ],
    }, indent=2))


if __name__ == "__main__":
    main()
