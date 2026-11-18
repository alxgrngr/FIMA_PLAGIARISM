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

PROCESSOR_QUERY = """
query($account:String!, $since:Time!) {
  viewer { accounts(filter:{accountTag:$account}) {
    durableObjectsInvocationsAdaptiveGroups(limit:1000, filter:{
      datetime_geq:$since, scriptName:"nusa-receipts-staging"
    }) {
      sum { requests errors }
      quantiles { cpuTimeP50 cpuTimeP99 wallTimeP50 wallTimeP99 }
      dimensions { datetime status }
    }
  } }
}
"""


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--since", required=True, help="UTC ISO timestamp")
    parser.add_argument("--processor", action="store_true", help="Query the Durable Object invocations")
    args = parser.parse_args()
    since = datetime.datetime.fromisoformat(args.since.replace("Z", "+00:00"))
    if since.tzinfo is None:
        parser.error("--since must include a timezone")
    since = since.astimezone(datetime.timezone.utc).isoformat()
    request = urllib.request.Request(
        "https://api.cloudflare.com/client/v4/graphql",
        data=json.dumps({"query": PROCESSOR_QUERY if args.processor else QUERY, "variables": {
            "account": os.environ["CLOUDFLARE_ACCOUNT_ID"], "since": since,
        }}).encode(),
        headers={"Authorization": "Bearer " + os.environ["CLOUDFLARE_API_TOKEN"],
                 "Content-Type": "application/json"},
    )
    with urllib.request.urlopen(request, timeout=30) as response:
        result = json.load(response)
    if result.get("errors"):
        raise SystemExit("Cloudflare rejected the metrics query; no CPU result recorded.")
    dataset = ("durableObjectsInvocationsAdaptiveGroups" if args.processor
               else "workersInvocationsAdaptive")
    rows = result["data"]["viewer"]["accounts"][0][dataset]
    print(json.dumps({
        "worker": "nusa-receipts-staging", "since": since,
        "dataset": dataset,
        "cpu_unit": "microseconds", "groups": rows,
        "limitations": [
            "Adaptive analytics; groups are not guaranteed to represent individual requests.",
            "No URL paths; only the Worker dataset includes the HTTP response status.",
            "Results may lag; up to 1000 groups. Not a load test or automatic deployment approval.",
        ],
    }, indent=2))


if __name__ == "__main__":
    main()
