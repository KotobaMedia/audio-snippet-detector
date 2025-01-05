import {it} from 'node:test';
import assert from 'node:assert';

import {AudioSnippetDetector} from './index';

it('should start up and shut down normally', async () => {
  const detector = new AudioSnippetDetector();
  detector.write.close();
  for await (const _ of detector) {
    assert.fail('should not have any data');
  }
  assert.ok(true);
});
