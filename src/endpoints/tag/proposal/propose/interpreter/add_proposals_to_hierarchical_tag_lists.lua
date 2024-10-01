redis.call('ZADD', KEYS[1], ARGV[1], ARGV[2])
redis.call('ZADD', KEYS[2], ARGV[3], ARGV[4])