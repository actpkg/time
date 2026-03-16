POST {{baseurl}}/tools/parse-datetime
{"arguments": {"input": "2024-06-15T10:30:00+00:00"}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" contains "2024"
jsonpath "$.content[0].data" contains "\"month\":6"
