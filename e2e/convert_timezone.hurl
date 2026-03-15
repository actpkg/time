POST http://[::1]:{{port}}/tools/convert-timezone
{"arguments": {"time": "2024-01-01T12:00:00+00:00", "to_timezone": "America/New_York"}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" contains "-05:00"

# QUERY method
QUERY http://[::1]:{{port}}/tools/convert-timezone
{"arguments": {"time": "2024-01-01T12:00:00+00:00", "to_timezone": "Europe/London"}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" contains "2024-01-01"
