/**
 * The theme as the backend hands it over (SPEC section 9.3).
 *
 * Mirrors the model in `src-tauri/src/theme/model.rs`. The backend has
 * already filled in defaults for anything a theme file left out, so every
 * field here is present.
 */

export type Base = 'dark' | 'light';
export type MeterVariant = 'bar' | 'segment' | 'needle' | 'line' | 'lcd';
export type SpectrumVariant = 'bar' | 'segment' | 'soft' | 'dots' | 'grass' | 'lcd' | 'sunken';
export type ProgressStyle = 'plain' | 'stars' | 'blocks';
export type PanelTexture = 'glass' | 'bezel' | 'brushed';
export type StageIndicator =
	| 'glow-chip'
	| 'outline-chip'
	| 'led'
	| 'glow-text'
	| 'brackets'
	| 'text'
	| 'check';
export type TimeDisplay = 'ghost-segment' | 'plain';
export type LabelCase = 'as-written' | 'uppercase';
export type ControlShape = 'soft' | 'square' | 'pill' | 'bevel' | 'ring' | 'box' | 'chrome' | 'glossy';
export type FrameStyle = 'hairline' | 'inset' | 'raised' | 'none' | 'card' | 'candy';
export type RowStyle = 'lines' | 'stripes' | 'plain';
export type Scrollbar = 'thin' | 'classic' | 'hidden';
export type TooltipStyle = 'plain' | 'panel';
export type DialogStyle = 'flat' | 'raised' | 'titled';
export type Easing = 'standard' | 'linear' | 'snap' | 'soft';
export type Density = 'comfortable' | 'compact';
export type ToneTarget = 'spectrum' | 'meter' | 'progress' | 'cover-glow' | 'backlight';

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
		/** Titles and headings. */
		display: string;
		/** The words of a song. */
		lyrics: string;
		/** A theme's own touches, such as a camcorder's time stamp. */
		accent: string;
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
			/** A vertical gradient under the whole window; empty when there is none. */
			backdrop: string[];
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
			/** Warning stripes, where a theme draws them. */
			stripe: string;
			/** Behind the line being sung; transparent when not marked. */
			highlight: string;
			/** The line being sung, written on the highlight. */
			highlightText: string;
			/** The floor under the transport, where a theme draws one. */
			ground: string;
			/** Ornament text; never the clip colour. */
			ornament: string;
			/** Something pale standing in a scene, and its edge. */
			paper: string;
			paperEdge: string;
			/** A liquid crystal display: its backlight, pixels and frame. */
			lcd: string;
			lcdInk: string;
			lcdEdge: string;
		};
		/** The two sides of a raised or sunken edge. */
		edge: {
			light: string;
			dark: string;
		};
		/** A window drawn inside the window (`chrome: window`); each list is a
		 *  gradient, top to bottom. */
		chrome: {
			outer: string;
			frame: string;
			title: string[];
			titleText: string;
			pane: string[];
			paneBox: string;
			paneText: string;
			taskbar: string[];
			taskbarText: string;
			start: string[];
			tray: string[];
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
		/** How the position bar of the transport is drawn. */
		progress: ProgressStyle;
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
	/** Ornament text, as dictionary keys (SPEC section 9.3, layer 7). */
	ornament: string[];
	/** Decorations, from the list Onsa knows how to draw. */
	decor: Decoration[];
	/** The theme's own arrangement of the window, if it has one. */
	signature: Signature | null;
}

export type Decoration =
	| 'rivets'
	| 'hazard-stripe'
	| 'marine-snow'
	| 'lure'
	| 'wallpaper'
	| 'carpet'
	| 'exit-sign'
	| 'clouds'
	| 'sparkles'
	| 'sky-eye'
	| 'brushed-lines'
	| 'hologram';
export type SidebarPlace = 'shown' | 'hidden';
export type QueuePlace = 'docked' | 'floating';
export type TransportStyle = 'full' | 'bar' | 'minimal';
export type LibraryHome = 'tracks' | 'albums';
export type NowPlayingStyle =
	| 'classic'
	| 'immersive'
	| 'instrument'
	| 'dive'
	| 'memo'
	| 'meadow'
	| 'pocket';
export type CoverFrame = 'plain' | 'instrument' | 'lit' | 'photo' | 'door' | 'disc';
export type PositionStyle = 'bar' | 'dial' | 'depth';
export type Chrome = 'none' | 'window' | 'corridor' | 'meadow';

/**
 * What goes where in the window (SPEC M16).
 *
 * The standard presets are values of this shape written in the interface;
 * a signature layout is one more, read from a theme. Nothing in it names a
 * theme: each field chooses among pieces any theme may use.
 */
export interface Arrangement {
	sidebar: SidebarPlace;
	queue: QueuePlace;
	transport: TransportStyle;
	signal: boolean;
	libraryHome: LibraryHome;
	nowPlaying: NowPlayingStyle;
	cover: CoverFrame;
	position: PositionStyle;
	chrome: Chrome;
	plates: boolean;
	/** Which Now Playing panels show, as `setPanels` takes them. */
	panels: Record<string, boolean>;
	/** Sizes that set a layout apart from the style it is drawn in, in CSS
	 *  pixels; what is not given keeps the style's own. */
	sizes?: {
		cover?: number | null;
		title?: number | null;
		lyrics?: number | null;
		spectrum?: number | null;
	};
}

/** A theme's own layout. */
export interface Signature {
	id: string;
	name: { id: string; en: string };
	arrangement: Arrangement;
}
