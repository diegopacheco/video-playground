local leases = KEYS[1]
local started = KEYS[2]
local device = ARGV[1]
local now = tonumber(ARGV[2])
local ttl = tonumber(ARGV[3])
local limit = tonumber(ARGV[4])
local policy = ARGV[5]

local expired = redis.call('ZRANGEBYSCORE', leases, '-inf', now)
if #expired > 0 then
  redis.call('ZREM', leases, unpack(expired))
  redis.call('HDEL', started, unpack(expired))
end

local expires = now + ttl

if redis.call('ZSCORE', leases, device) then
  redis.call('ZADD', leases, expires, device)
  redis.call('PEXPIRE', leases, ttl)
  redis.call('PEXPIRE', started, ttl)
  local since = redis.call('HGET', started, device) or ARGV[2]
  return {'granted', since, tostring(expires)}
end

local reply = {'granted', ARGV[2], tostring(expires)}
local count = redis.call('ZCARD', leases)
while count >= limit do
  if policy ~= 'kick_oldest' then
    return {'rejected', tostring(count)}
  end
  local all = redis.call('HGETALL', started)
  local victim = nil
  local victim_start = nil
  for i = 1, #all, 2 do
    local s = tonumber(all[i + 1])
    if victim == nil or s < victim_start or (s == victim_start and all[i] < victim) then
      victim = all[i]
      victim_start = s
    end
  end
  if victim == nil then
    return redis.error_reply('lease index out of sync')
  end
  redis.call('ZREM', leases, victim)
  redis.call('HDEL', started, victim)
  table.insert(reply, victim)
  count = count - 1
end

redis.call('ZADD', leases, expires, device)
redis.call('HSET', started, device, ARGV[2])
redis.call('PEXPIRE', leases, ttl)
redis.call('PEXPIRE', started, ttl)
return reply
