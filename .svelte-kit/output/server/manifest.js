export const manifest = (() => {
function __memo(fn) {
	let value;
	return () => value ??= (value = fn());
}

return {
	appDir: "_app",
	appPath: "_app",
	assets: new Set(["robots.txt"]),
	mimeTypes: {".txt":"text/plain"},
	_: {
		client: {start:"_app/immutable/entry/start.BTAae89A.js",app:"_app/immutable/entry/app.h5LfaFnt.js",imports:["_app/immutable/entry/start.BTAae89A.js","_app/immutable/chunks/BEQHYTzk.js","_app/immutable/chunks/D5QHtcP-.js","_app/immutable/chunks/Co_QHFx6.js","_app/immutable/entry/app.h5LfaFnt.js","_app/immutable/chunks/D5QHtcP-.js","_app/immutable/chunks/DaIjg0jD.js","_app/immutable/chunks/Bc0mxiO4.js","_app/immutable/chunks/Co_QHFx6.js","_app/immutable/chunks/DGfVVGiB.js"],stylesheets:[],fonts:[],uses_env_dynamic_public:false},
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
