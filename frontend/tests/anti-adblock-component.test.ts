import { cleanup, fireEvent, render } from '@testing-library/svelte';
import { afterEach, describe, expect, it, vi } from 'vitest';
import AntiAdblockSettings from '../src/lib/components/AntiAdblockSettings.svelte';

const site = {
  id: 'site-1',
  name: 'Example',
  domain: 'example.com',
  writeKey: 'd8f6f152-7a9e-4eb9-a8a1-468db4c0ea33',
  serverWriteKey: '7e55bd93-2601-46fc-881a-e847209f25f1',
  antiAdblockServer: 'caddy' as const,
  antiAdblockJsPath: '/456bbb63bb86.js',
  antiAdblockBeaconPath: '/0d31360a3101'
};

afterEach(cleanup);

describe('AntiAdblockSettings', () => {
  it('shows editable defaults, server configuration, minimal snippet, and test links', () => {
    const view = render(AntiAdblockSettings, { site, analyticsOrigin: 'https://slimlytics.com', save: vi.fn() });
    expect(view.getByLabelText('Server type')).toHaveValue('caddy');
    expect(view.getByLabelText('JavaScript path')).toHaveValue('/456bbb63bb86.js');
    expect(view.getByLabelText('Beacon path')).toHaveValue('/0d31360a3101');
    expect(view.getByText(/handle \/456bbb63bb86\.js/)).toBeInTheDocument();
    expect(view.getByText('<script async src="/456bbb63bb86.js"></script>')).toBeInTheDocument();
    expect(view.getByRole('link', { name: 'Test JavaScript path' })).toHaveAttribute('href', 'https://example.com/456bbb63bb86.js');
    expect(view.getByRole('link', { name: 'Test beacon path' })).toHaveAttribute('href', 'https://example.com/0d31360a3101');
  });

  it('saves the selected server and edited paths', async () => {
    const save = vi.fn().mockResolvedValue(undefined);
    const view = render(AntiAdblockSettings, { site, analyticsOrigin: 'https://slimlytics.com', save });
    await fireEvent.change(view.getByLabelText('Server type'), { target: { value: 'nginx' } });
    await fireEvent.input(view.getByLabelText('JavaScript path'), { target: { value: '/newscript123.js' } });
    await fireEvent.input(view.getByLabelText('Beacon path'), { target: { value: '/newbeacon123' } });
    await fireEvent.click(view.getByRole('button', { name: 'Save configuration' }));
    expect(save).toHaveBeenCalledWith({ serverType: 'nginx', jsPath: '/newscript123.js', beaconPath: '/newbeacon123' });
    expect(await view.findByText('Configuration saved.')).toBeInTheDocument();
  });
});
