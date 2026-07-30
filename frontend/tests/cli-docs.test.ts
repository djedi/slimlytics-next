import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import CliDocs from '../src/lib/components/CliDocs.svelte';
import { cliCommands, installCommand, type CliCommandDoc } from '../src/lib/cli-docs';

const requiredCommands = [
  'slimlytics auth login',
  'slimlytics auth use-token',
  'slimlytics auth status',
  'slimlytics auth logout',
  'slimlytics account show',
  'slimlytics token list',
  'slimlytics token revoke',
  'slimlytics site list',
  'slimlytics site show',
  'slimlytics site add',
  'slimlytics site ensure',
  'slimlytics site delete',
  'slimlytics tracking show',
  'slimlytics tracking configure'
];

describe('CLI documentation', () => {
  it('documents installation and every command in the shipped CLI', () => {
    const documented = cliCommands.map((command) => command.usage);
    for (const command of requiredCommands) {
      expect(documented.some((usage) => usage.startsWith(command))).toBe(true);
    }
    expect(installCommand).toContain('cli-v0.2.0');
    const useToken = cliCommands.find((command: CliCommandDoc) => command.usage.startsWith('slimlytics auth use-token'));
    expect(useToken?.details).toMatch(/piped standard input/i);
    expect(useToken?.details).toMatch(/does not prompt/i);
  });

  it('renders public navigation, security guidance, JSON automation, and all commands', () => {
    render(CliDocs);
    expect(screen.getByRole('heading', { name: 'Slimlytics CLI' })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Interactive API reference' })).toHaveAttribute('href', '/api/docs');
    expect(document.body).toHaveTextContent(/SlimToolkit already owns the popular slim command/i);
    expect(document.body).toHaveTextContent(/SLIMLYTICS_TOKEN/);
    for (const command of requiredCommands) {
      expect(screen.getAllByText((_, element) => element?.tagName === 'CODE' && element.textContent?.startsWith(command) === true).length).toBeGreaterThan(0);
    }
  });
});
