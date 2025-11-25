export const manifest = (() => {
function __memo(fn) {
	let value;
	return () => value ??= (value = fn());
}

return {
	appDir: "_app",
	appPath: "_app",
	assets: new Set(["vite.svg"]),
	mimeTypes: {".svg":"image/svg+xml"},
	_: {
		client: {start:"_app/immutable/entry/start.BGUap110.js",app:"_app/immutable/entry/app.CjWL8MbV.js",imports:["_app/immutable/entry/start.BGUap110.js","_app/immutable/chunks/CfwBsxvx.js","_app/immutable/chunks/70-DPiHw.js","_app/immutable/chunks/DTWOZuC0.js","_app/immutable/entry/app.CjWL8MbV.js","_app/immutable/chunks/70-DPiHw.js","_app/immutable/chunks/DwE8ATd1.js","_app/immutable/chunks/BfH4tdX4.js","_app/immutable/chunks/DOFZFML0.js","_app/immutable/chunks/DTWOZuC0.js","_app/immutable/chunks/DCy56RVY.js"],stylesheets:[],fonts:[],uses_env_dynamic_public:false},
		nodes: [
			__memo(() => import('./nodes/0.js')),
			__memo(() => import('./nodes/1.js')),
			__memo(() => import('./nodes/2.js'))
		],
		remotes: {
			
		},
		routes: [
			{
				id: "/",
				pattern: /^\/$/,
				params: [],
				page: { layouts: [0,], errors: [1,], leaf: 2 },
				endpoint: null
			}
		],
		prerendered_routes: new Set([]),
		matchers: async () => {
			
			return {  };
		},
		server_assets: {}
	}
}
})();
