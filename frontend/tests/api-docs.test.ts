import { render, waitFor } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import ApiDocs from '../src/lib/components/ApiDocs.svelte';

const createApiReference = vi.fn();
vi.mock('@scalar/api-reference', () => ({ createApiReference }));

describe('ApiDocs', () => {
  beforeEach(() => createApiReference.mockClear());

  it('loads the locally bundled Scalar reference against the public OpenAPI document', async () => {
    const { container } = render(ApiDocs);
    expect(container.querySelector('#scalar-api-reference')).toBeInTheDocument();
    await waitFor(() => expect(createApiReference).toHaveBeenCalledOnce());
    expect(createApiReference).toHaveBeenCalledWith(
      '#scalar-api-reference',
      expect.objectContaining({
        url: '/api/openapi.json',
        telemetry: false,
        withDefaultFonts: false
      })
    );
  });
});
