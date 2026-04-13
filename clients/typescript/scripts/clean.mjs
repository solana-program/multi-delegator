import { rmSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const root = dirname(fileURLToPath(import.meta.url));

rmSync(join(root, '..', 'dist'), { recursive: true, force: true });
rmSync(join(root, '..', 'src', 'generated'), { recursive: true, force: true });
