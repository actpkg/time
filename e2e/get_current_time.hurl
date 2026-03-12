# UTC time
POST {{host}}/tools/get-current-time
{"arguments": {}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" contains "T"

# With timezone
POST {{host}}/tools/get-current-time
{"arguments": {"timezone": "Europe/Moscow"}}
HTTP 200
[Asserts]
jsonpath "$.content[0].data" matches "\\+0[34]:00"

# Invalid timezone
POST {{host}}/tools/get-current-time
{"arguments": {"timezone": "Not/A/Zone"}}
HTTP 422
[Asserts]
jsonpath "$.error.kind" == "std:invalid-args"
