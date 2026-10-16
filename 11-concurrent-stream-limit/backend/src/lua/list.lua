local leases = KEYS[1]
local started = KEYS[2]
local now = tonumber(ARGV[1])

local expired = redis.call('ZRANGEBYSCORE', leases, '-inf', now)
if #expired > 0 then
  redis.call('ZREM', leases, unpack(expired))
  redis.call('HDEL', started, unpack(expired))
end

local rows = redis.call('ZRANGE', leases, 0, -1, 'WITHSCORES')
local out = {}
for i = 1, #rows, 2 do
  local device = rows[i]
  table.insert(out, device)
  table.insert(out, redis.call('HGET', started, device) or rows[i + 1])
  table.insert(out, rows[i + 1])
end
return out
