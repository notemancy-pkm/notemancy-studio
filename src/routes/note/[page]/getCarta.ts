// carta-instance.ts
import { Carta, Markdown, type Plugin } from "carta-md";
import min_light from "shiki/themes/min-light.mjs";
import min_dark from "shiki/themes/min-dark.mjs";
import "rehype-callouts/theme/vitepress";
import { svelteCustom } from "@cartamd/plugin-component/svelte";
import { initializeComponents } from "@cartamd/plugin-component/svelte";
import { component } from "@cartamd/plugin-component";
import rehypeKatex from "rehype-katex";
import "katex/dist/katex.css";
import { math } from "@cartamd/plugin-math";
import { anchor } from "@cartamd/plugin-anchor";
import { code } from "@cartamd/plugin-code";
import DOMPurify from "isomorphic-dompurify";
import rehypeCallouts from "rehype-callouts";
import rehypeMermaid from "rehype-mermaid";
import cartawiki from "./cartawiki";
import WikiLinkPreview from "./WikiLinkPreview.svelte";

let cachedCarta: Carta | null = null;

const mermaid: Plugin = {
	transformers: [
		{
			execution: "async",
			type: "rehype",
			transform({ processor }) {
				processor.use(rehypeMermaid, { strategy: "img-png" });
			},
		},
	],
};

const callouts: Plugin = {
	transformers: [
		{
			execution: "async",
			type: "rehype",
			transform({ processor }) {
				processor.use(rehypeCallouts);
			},
		},
	],
};

const mapped = [
	svelteCustom(
		"wiki-link",
		(node) =>
			node.tagName === "a" &&
			node.properties &&
			Array.isArray(node.properties.className) &&
			node.properties.className.includes("wiki-link"),
		WikiLinkPreview,
	),
];

export function getCartaInstance(theme: "dark" | "light", forceReinit = false) {
	if (!cachedCarta || forceReinit) {
		cachedCarta = new Carta({
			theme: theme === "dark" ? min_dark : min_light,
			shikiOptions: { themes: [min_light, min_dark] },
			extensions: [
				cartawiki,
				component(mapped, initializeComponents),
				math(),
				callouts,
				mermaid,
				anchor(),
				code({
					langs: [
						"javascript",
						"docker",
						"py",
						"markdown",
						"yaml",
						"toml",
						"bash",
					],
				}),
			],
			sanitizer: DOMPurify.sanitize,
		});
	}
	return cachedCarta;
}
