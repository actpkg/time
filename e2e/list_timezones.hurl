POST {{host}}/tools/list-timezones
{"arguments": {"filter": "moscow"}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" contains "Moscow"
