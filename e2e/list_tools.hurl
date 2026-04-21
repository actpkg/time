POST {{baseurl}}/tools
HTTP 200
[Asserts]
jsonpath "$.tools" count == 1
jsonpath "$.tools[0].name" "get_current_time"
