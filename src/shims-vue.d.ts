declare module '*.vue' {
  import type { DefineComponent } from 'vue'
  const component: DefineComponent<{}, {}, any>
  export default component
}

declare module '*.scss' {
  const content: any
  export default content
}

declare module '*.css' {
  const content: any
  export default content
}

declare module '*.png' {
  const content: string
  export default content
}

declare module '*.jpg' {
  const content: string
  export default content
}

declare module '*.jpeg' {
  const content: string
  export default content
}

declare module '*.gif' {
  const content: string
  export default content
}

declare module '*.svg' {
  const content: string
  export default content
}

declare module '*.ico' {
  const content: string
  export default content
}

declare module '*.woff' {
  const content: string
  export default content
}

declare module '*.woff2' {
  const content: string
  export default content
}

declare module '*.ttf' {
  const content: string
  export default content
}

declare module '*.eot' {
  const content: string
  export default content
}

// vuex 类型声明（exports 兼容）
declare module 'vuex' {
  export * from 'vuex/types/index.d.ts';
}

// vue3-eventbus 类型声明
declare module 'vue3-eventbus' {
  import mitt from 'mitt';
  const bus: mitt.Emitter<any> & { install: (app: any, options?: any) => any };
  export default bus;
  export { bus };
}
