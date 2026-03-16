POST {{baseurl}}/tools
HTTP 200
[Asserts]
jsonpath "$.tools" count >= 5
jsonpath "$.tools[*].name" contains "get-current-time"
jsonpath "$.tools[*].name" contains "convert-timezone"
jsonpath "$.tools[*].name" contains "date-arithmetic"
jsonpath "$.tools[*].name" contains "list-timezones"
jsonpath "$.tools[*].name" contains "parse-datetime"

# QUERY method (safe, cacheable)
QUERY {{baseurl}}/tools
HTTP 200
[Asserts]
jsonpath "$.tools" count >= 5
