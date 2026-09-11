/**
 * Turns a theme into what the stylesheets read: `--onsa-*` custom properties
 * for the tokens, and `data-onsa-*` attributes for the component variants
 * (SPEC section 9.3).
 */

import type { Theme } from './types';

/**
 * Builds the custom properties of a theme.
 *
 * Exported on its own so it can be read without a document.
 */
export function themeVariables(theme: Theme): Record<string, string> {
	const { color, shape, fonts, effects, toneColor } = theme;
	return {
		'--onsa-surface-app': color.surface.app,
		'--onsa-surface-body': color.surface.body,
		'--onsa-surface-body-highlight': color.surface.bodyHighlight,
		'--onsa-surface-well': color.surface.well,
		'--onsa-surface-raised': color.surface.raised,
		'--onsa-surface-line': color.surface.line,

		'--onsa-text-primary': color.text.primary,
		'--onsa-text-secondary': color.text.secondary,
		'--onsa-text-silkscreen': color.text.silkscreen,

		'--onsa-role-label': color.role.label,
		'--onsa-role-adjustable': color.role.adjustable,
		'--onsa-role-active': color.role.active,
		'--onsa-role-position': color.role.position,
		'--onsa-role-caution': color.role.caution,
		'--onsa-role-clip': color.role.clip,

		'--onsa-lit-on': color.lit.on,
		'--onsa-lit-ghost': color.lit.ghost,
		'--onsa-lit-glow': color.lit.glow,
		'--onsa-lit-secondary': color.lit.secondary,
		'--onsa-lit-meter-face': color.lit.meterFace,
		'--onsa-lit-needle': color.lit.needle,
		'--onsa-lit-knob': color.lit.knob,
		'--onsa-lit-led': color.lit.led,

		'--onsa-radius-sm': `${shape.radius.sm}px`,
		'--onsa-radius-md': `${shape.radius.md}px`,
		'--onsa-radius-lg': `${shape.radius.lg}px`,
		'--onsa-hairline': `${shape.hairline}px`,

		'--onsa-font-ui': fontStack(fonts.ui, fonts.cjkFallback),
		'--onsa-font-label': fontStack(fonts.label, fonts.cjkFallback),
		'--onsa-font-numeric': fontStack(fonts.numeric, fonts.cjkFallback, 'ui-monospace, monospace'),

		'--onsa-grain': `${effects.grain}`,
		'--onsa-tone-range': `${toneColor.range}deg`
	};
}

/** Builds the `data-onsa-*` attributes of a theme. */
export function themeAttributes(theme: Theme): Record<string, string> {
	return {
		'data-onsa-theme': theme.id,
		'data-onsa-base': theme.base,
		'data-onsa-meter': theme.variants.meter,
		'data-onsa-spectrum': theme.variants.spectrum,
		'data-onsa-panel-texture': theme.variants.panelTexture,
		'data-onsa-stage-indicator': theme.variants.stageIndicator,
		'data-onsa-time-display': theme.variants.timeDisplay,
		'data-onsa-density': theme.shape.density,
		'data-onsa-label-case': theme.effects.labelCase,
		'data-onsa-glow': String(theme.effects.glow),
		'data-onsa-glass-reflection': String(theme.effects.glassReflection),
		'data-onsa-tone-color': String(theme.toneColor.enabled)
	};
}

/** Dresses the document in a theme. */
export function applyTheme(theme: Theme, root: HTMLElement): void {
	for (const [property, value] of Object.entries(themeVariables(theme))) {
		root.style.setProperty(property, value);
	}
	for (const [attribute, value] of Object.entries(themeAttributes(theme))) {
		root.setAttribute(attribute, value);
	}
	root.style.colorScheme = theme.base;
}

/**
 * A font stack: the family the theme asks for, then the fallback for
 * Japanese, Korean and Chinese, then whatever the system offers. Every family
 * is bundled with the application; nothing is ever loaded from the internet
 * (SPEC section 14).
 */
function fontStack(family: string, cjkFallback: string, generic = 'system-ui, sans-serif'): string {
	const quoted = [family, cjkFallback]
		.filter((name) => name.trim().length > 0)
		.map((name) => (/^[A-Za-z][A-Za-z0-9\s-]*$/.test(name.trim()) ? `"${name.trim()}"` : ''))
		.filter((name) => name.length > 0);
	return [...quoted, generic].join(', ');
}
