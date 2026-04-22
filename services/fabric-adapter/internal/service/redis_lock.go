package service

import (
	"context"
	"fmt"
	"strings"
	"time"

	"github.com/redis/go-redis/v9"
)

type RedisShortLock struct {
	client    *redis.Client
	namespace string
	ttl       time.Duration
}

func NewRedisShortLock(
	ctx context.Context,
	redisURL string,
	namespace string,
	ttl time.Duration,
) (*RedisShortLock, error) {
	options, err := redis.ParseURL(redisURL)
	if err != nil {
		return nil, fmt.Errorf("parse REDIS_URL: %w", err)
	}
	client := redis.NewClient(options)
	if err := client.Ping(ctx).Err(); err != nil {
		_ = client.Close()
		return nil, fmt.Errorf("ping Redis: %w", err)
	}
	if ttl <= 0 {
		ttl = 15 * time.Second
	}
	return &RedisShortLock{
		client:    client,
		namespace: strings.TrimSuffix(namespace, ":"),
		ttl:       ttl,
	}, nil
}

func (lock *RedisShortLock) Acquire(
	ctx context.Context,
	eventID string,
) (string, bool, error) {
	key := lock.LockKey(eventID)
	acquired, err := lock.client.SetNX(ctx, key, "1", lock.ttl).Result()
	if err != nil {
		return key, false, fmt.Errorf("set redis lock: %w", err)
	}
	return key, acquired, nil
}

func (lock *RedisShortLock) Release(ctx context.Context, eventID string) error {
	key := lock.LockKey(eventID)
	if err := lock.client.Del(ctx, key).Err(); err != nil {
		return fmt.Errorf("release redis lock: %w", err)
	}
	return nil
}

func (lock *RedisShortLock) Exists(ctx context.Context, eventID string) (bool, error) {
	key := lock.LockKey(eventID)
	exists, err := lock.client.Exists(ctx, key).Result()
	if err != nil {
		return false, fmt.Errorf("lookup redis lock: %w", err)
	}
	return exists > 0, nil
}

func (lock *RedisShortLock) LockKey(eventID string) string {
	return fmt.Sprintf("%s:fabric-adapter:consumer-lock:%s", lock.namespace, eventID)
}

func (lock *RedisShortLock) Close() error {
	return lock.client.Close()
}
