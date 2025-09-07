// load-tests/scenarios.js
import http from 'k6/http';
import { check, sleep } from 'k6';
import { randomIntBetween } from 'https://jslib.k6.io/k6-utils/1.2.0/index.js';

export const options = {
  stages: [
    { duration: '30s', target: 100 },   // Ramp-up to 100 users
    { duration: '2m', target: 100 },    // Stay at 100 users
    { duration: '30s', target: 1000 },  // Ramp-up to 1000 users
    { duration: '5m', target: 1000 },   // Stay at 1000 users
    { duration: '30s', target: 0 },     // Ramp-down to 0
  ],
  thresholds: {
    'http_req_duration': ['p(95)<200'], // 95% of requests < 200ms
    'http_req_failed': ['rate<0.01'],   // Error rate < 1%
  },
};

const BASE_URL = 'http://localhost:3000';
const JWT_SECRET = 'jwt_secret_32_chars_min';

// Generate test users
const testUsers = [];
for (let i = 0; i < 100; i++) {
  testUsers.push({
    mobile: `+9198765${String(43210 + i).padStart(5, '0')}`,
    userId: `user_${i}`,
    token: null,
  });
}

export default function () {
  const user = testUsers[randomIntBetween(0, testUsers.length - 1)];
  
  if (!user.token) {
    // Register + Login
    registerAndLogin(user);
  }

  // 70% chance: Check balance
  if (Math.random() < 0.7) {
    checkBalance(user);
  }

  // 30% chance: Make payment
  if (Math.random() < 0.3) {
    makePayment(user);
  }

  sleep(randomIntBetween(1, 5));
}

function registerAndLogin(user) {
  // Register
  let res = http.post(`${BASE_URL}/auth/register`, {
    mobile: user.mobile,
  });
  check(res, { 'Register status 200': (r) => r.status === 200 });

  // In real test: wait for OTP or use test endpoint
  // For demo: assume OTP is 123456
  res = http.post(`${BASE_URL}/auth/verify-otp`, {
    mobile: user.mobile,
    otp: '123456',
  });
  check(res, { 'Login status 200': (r) => r.status === 200 });
  
  if (res.status === 200) {
    user.token = res.json('access_token');
  }
}

function checkBalance(user) {
  const res = http.get(`${BASE_URL}/wallet/balance`, {
    headers: { Authorization: `Bearer ${user.token}` },
  });
  check(res, {
    'Balance status 200': (r) => r.status === 200,
    'Balance is number': (r) => typeof r.json() === 'number',
  });
}

function makePayment(user) {
  // Pay to random user
  const toUser = testUsers[randomIntBetween(0, testUsers.length - 1)];
  if (toUser.mobile === user.mobile) return;

  const res = http.post(`${BASE_URL}/pay/phone`, {
    to_mobile: toUser.mobile,
    amount: randomIntBetween(1000, 50000), // ₹10 to ₹500
    idempotency_key: `k6_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
  }, {
    headers: { Authorization: `Bearer ${user.token}` },
  });
  check(res, {
    'Payment status 200': (r) => r.status === 200,
    'Payment success': (r) => r.json('status') === 'Success',
  });
}