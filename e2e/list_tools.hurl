POST {{host}}/tools
HTTP 200
[Asserts]
jsonpath "$.tools" count >= 5
jsonpath "$.tools[*].name" includes "get-current-time"
jsonpath "$.tools[*].name" includes "convert-timezone"
jsonpath "$.tools[*].name" includes "date-arithmetic"
jsonpath "$.tools[*].name" includes "list-timezones"
jsonpath "$.tools[*].name" includes "parse-datetime"
