local leases = KEYS[1]
local started = KEYS[2]
local device = ARGV[1]
local now = tonumber(ARGV[2])
local ttl = tonumber(ARGV[3])

local expired = redis.call('ZRANGEBYSCORE', leases, '-inf', now)
if #expired > 0 then
  redis.call('ZREM', leases, unpack(expired))
  redis.call('HDEL', started, unpack(expired))
end

if not redis.call('ZSCORE', leases, device) then
  return {}
end

local expires = now + ttl
redis.call('ZADD', leases, 'XX', expires, device)
redis.call('PEXPIRE', leases, ttl)
redis.call('PEXPIRE', started, ttl)
local since = redis.call('HGET', started, device) or ARGV[2]
return {since, tostring(expires)}
