package cache

import (
	"context"
	redis "github.com/redis/go-redis/v9"
	"os"
	"strings"
	"time"
)

type RedisClient struct {
	Client *redis.Client
}

// NewRedisClient initializes a Redis client with sensible defaults, reading REDIS_URL if provided.
func NewRedisClient() (*RedisClient, error) {
	// Determine URL: env var or default
	url := os.Getenv("REDIS_URL")
	if url == "" {
		// Default to localhost
		url = "redis://localhost:6379"
	}

	var opt *redis.Options
	// Try to parse if URL is a redis:// style
	if strings.HasPrefix(url, "redis://") {
		if parsed, err := redis.ParseURL(url); err == nil {
			parsed.PoolSize = 10
			parsed.MinIdleConns = 5
			opt = parsed
		} else {
			opt = &redis.Options{Addr: url, PoolSize: 10, MinIdleConns: 5}
		}
	} else {
		// Fallback: treat as address
		opt = &redis.Options{Addr: url, PoolSize: 10, MinIdleConns: 5}
	}

	client := redis.NewClient(opt)
	// Quick health check
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	if err := client.Ping(ctx).Err(); err != nil {
		// Return client anyway; caller can retry/handle error
		return &RedisClient{Client: client}, err
	}
	return &RedisClient{Client: client}, nil
}

func (r *RedisClient) Get(ctx context.Context, key string) (string, error) {
	ctx, cancel := context.WithTimeout(ctx, 5*time.Second)
	defer cancel()
	return r.Client.Get(ctx, key).Result()
}

func (r *RedisClient) Set(ctx context.Context, key string, value interface{}, ttl time.Duration) error {
	ctx, cancel := context.WithTimeout(ctx, 5*time.Second)
	defer cancel()
	return r.Client.Set(ctx, key, value, ttl).Err()
}

func (r *RedisClient) Delete(ctx context.Context, keys ...string) error {
	ctx, cancel := context.WithTimeout(ctx, 5*time.Second)
	defer cancel()
	return r.Client.Del(ctx, keys...).Err()
}

func (r *RedisClient) Close() error {
	return r.Client.Close()
}
