import {it} from 'node:test';
import assert from 'node:assert';
import fs from 'node:fs';
import {pipeline} from 'node:stream/promises';

import {AudioSnippetDetector} from './index';

interface AsyncIterableItem<T> {
  [Symbol.asyncIterator](): AsyncIterator<T>;
}

async function take<T>(asyncIterable: AsyncIterableItem<T>, n: number): Promise<T[]> {
  const results: T[] = [];
  for await (const item of asyncIterable) {
    results.push(item);
    if (results.length >= n) {
      break;
    }
  }
  return results;
}

it('should start up and shut down normally', async () => {
  const detector = new AudioSnippetDetector();
  detector.end();
  for await (const _ of detector) {
    assert.fail('should not have any data');
  }
  assert.ok(true);
});

it('recognizes a known snippet', async () => {
  const detector = new AudioSnippetDetector();

  const chimeData = await fs.promises.readFile('test/db/chime.raw');
  detector.add_database('chime', chimeData);

  const audioData = fs.createReadStream('test/chime-01.raw');
  const pipelinePromise = pipeline(audioData, detector);
  const items = await take(detector, 5);
  assert.strictEqual(items.length, 5);
  for (const item of items) {
    assert.strictEqual(item.label, 'chime');
    assert.ok(item.score >= 0.9);
  }
  await pipelinePromise;
  assert.ok(true);
});
