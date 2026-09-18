/**
 * The theme as the backend hands it over (SPEC section 9.3).
 *
 * Mirrors the model in `src-tauri/src/theme/model.rs`. The backend has
 * already filled in defaults for anything a theme file left out, so every
 * field here is present.
 */

export type Base = 'dark' | 'light';
export type MeterVariant = 'bar' | 'segment' | 'needle';
export type SpectrumVariant = 'bar' | 'segment' | 'soft';
export type PanelTexture = 'glass' | 'bezel' | 'brushed';
export type StageIndicator = 'glow-chip' | 'outline-chip' | 'led';
export type TimeDisplay = 'ghost-segment' | 'plain';
export type LabelCase = 'as-written' | 'uppercase';
export type ControlShape = 'soft' | 'square' | 'pill' | 'bevel';
export type FrameStyle = 'hairline' | 'inset' | 'raised' | 'none';
export type RowStyle = 'lines' | 'stripes' | 'plain';
export type Scrollbar = 'thin' | 'classic' | 'hidden';
export type TooltipStyle = 'plain' | 'panel';
export type DialogStyle = 'flat' | 'raised' | 'titled';
export type Easing = 'standard' | 'linear' | 'snap' | 'soft';
export type Density = 'comfortable' | 'compact';
export type ToneTarget = 'spectrum' | 'meter' | 'progress' | 'cover-glow';

export interface Theme {
	id: string;
	name: { id: string; en: string };
	version: number;
	builtin: boolean;
	base: Base;
	fonts: {
		ui: string;
		numeric: string;
		label: string;
		cjkFallback: string;
	};
	color: {
		surface: {
			app: string;
			body: string;
			bodyHighlight: string;
			well: string;
			raised: string;
			line: string;
		};
		text: {
			primary: string;
			secondary: string;
			silkscreen: string;
		};
		/** Components address these roles, never a raw colour. */
		role: {
			label: string;
			adjustable: string;
			active: string;
			position: string;
			caution: string;
			clip: string;
		};
		lit: {
			on: string;
			ghost: string;
			glow: string;
			secondary: string;
			meterFace: string;
			needle: string;
			knob: string;
			led: string;
		};
		/** The two sides of a raised or sunken edge. */
		edge: {
			light: string;
			dark: string;
		};
	};
	shape: {
		radius: { sm: number; md: number; lg: number };
		density: Density;
		hairline: number;
		control: ControlShape;
		frame: FrameStyle;
	};
	variants: {
		meter: MeterVariant;
		spectrum: SpectrumVariant;
		panelTexture: PanelTexture;
		stageIndicator: StageIndicator;
		timeDisplay: TimeDisplay;
		rows: RowStyle;
		scrollbar: Scrollbar;
		tooltip: TooltipStyle;
		dialog: DialogStyle;
	};
	effects: {
		glow: boolean;
		glassReflection: boolean;
		grain: number;
		labelCase: LabelCase;
	};
	/** How long things take to move, and how they move. */
	motion: {
		fast: number;
		slow: number;
		ease: Easing;
	};
	toneColor: {
		enabled: boolean;
		targets: ToneTarget[];
		/** Where heavy sound leans. */
		warm: string;
		/** Where bright sound leans. */
		cool: string;
	};
}
