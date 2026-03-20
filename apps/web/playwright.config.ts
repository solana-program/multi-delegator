import { defineConfig } from '@playwright/test';
import * as dotenv from 'dotenv';
import * as path from 'path';

dotenv.config({ path: path.resolve(__dirname, '../../.env') });

export default defineConfig({
    projects: [
        {
            name: 'chromium',
            use: { channel: 'chromium' },
        },
    ],
    reporter: [['list'], ['html', { open: 'never' }]],
    retries: 0,
    testDir: './e2e',
    timeout: 60_000,
    use: {
        baseURL: process.env.APP_URL ?? 'http://localhost:3000',
        headless: true,
        screenshot: 'only-on-failure',
        video: 'retain-on-failure',
    },
    workers: 1,
});
