---
name: time
description: Get current time in any timezone
metadata:
  act: {}
---

# Time Component

Get current date and time. Use when you need to know what time it is now.

## Tool: get_current_time

Returns current time as RFC 3339 string.

```
get_current_time()
→ "2026-03-31T20:15:00+00:00"

get_current_time(timezone: "America/New_York")
→ "2026-03-31T16:15:00-04:00"

get_current_time(timezone: "Asia/Tokyo")
→ "2026-04-01T05:15:00+09:00"
```

Timezone is optional (defaults to UTC). Use IANA timezone names.
