GET {{host}}/info
HTTP 200
[Asserts]
jsonpath "$.name" == "time"
jsonpath "$.version" isString
