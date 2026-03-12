POST {{host}}/tools/date-arithmetic
{"arguments": {"datetime": "2024-01-01T00:00:00+00:00", "operation": "add", "days": 10}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" contains "2024-01-11"
