declare module "*.vue" {
    import type { DefineComponent } from "vue"
    const component: DefineComponent<Record<string, never>, Record<string, never>, any>
    export default component
}

// the git commit the frontend was built from, see vite.config.ts
declare const __BUILD_COMMIT__: string;
