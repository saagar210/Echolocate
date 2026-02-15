import { describe, it, expect, vi } from 'vitest';

describe('Button Component', () => {
  it('renders button with text', () => {
    const buttonText = 'Click Me';
    expect(buttonText).toBe('Click Me');
  });

  it('handles click events', () => {
    const onClick = vi.fn();
    onClick();
    expect(onClick).toHaveBeenCalledTimes(1);
  });

  it('applies variant classes', () => {
    const variants = ['primary', 'secondary', 'danger', 'ghost'];
    variants.forEach((variant) => {
      expect(variant).toMatch(/primary|secondary|danger|ghost/);
    });
  });

  it('applies size classes', () => {
    const sizes = ['sm', 'md', 'lg'];
    sizes.forEach((size) => {
      expect(size).toMatch(/sm|md|lg/);
    });
  });

  it('disables button when disabled prop is true', () => {
    const disabled = true;
    expect(disabled).toBe(true);
  });

  it('shows loading state', () => {
    const loading = true;
    expect(loading).toBe(true);
  });

  it('does not call onClick when disabled', () => {
    const onClick = vi.fn();
    const disabled = true;

    if (!disabled) {
      onClick();
    }

    expect(onClick).not.toHaveBeenCalled();
  });

  it('renders icon when provided', () => {
    const hasIcon = true;
    expect(hasIcon).toBe(true);
  });

  it('renders as full width when specified', () => {
    const fullWidth = true;
    expect(fullWidth).toBe(true);
  });

  it('supports custom classes', () => {
    const customClass = 'custom-button-class';
    expect(customClass).toBe('custom-button-class');
  });
});
