# ctx-provider

> React hooks and context utils

[![NPM](https://img.shields.io/npm/v/ctx-provider.svg)](https://www.npmjs.com/package/ctx-provider) [![JavaScript Style Guide](https://img.shields.io/badge/code_style-standard-brightgreen.svg)](https://standardjs.com)

## Install

```bash
npm install --save ctx-provider
# or
yarn add ctx-provider
```

## Usage

Create a context.

```tsx
// src/context/count.js
import { useState } from 'react'
import createStore from 'ctx-provider'

const useCount = () => {
  const [count, setCount] = useState(0)

  const inc = () => setCount(count + 1)
  const dec = () => setCount(count - 1)

  return { count, inc, dec }
}

export const { ctx, Provider } = createStore(useCount)
```

Apply the provider to the app.

```tsx
// src/App.jsx
import React from 'react'

import { Provider as CountProvider } from './context/count'

const App = () => (
  <Provider>
    <Counter />
  </Provider>
)
```

Use the context from any component.

```tsx
// src/components/Counter.jsx
import React, { useContext } from 'react'
import { ctx as countContext } from './context/count'

const Counter = () => {
  const { count, inc, dec } = useContext(countContext)

  return (
    <div>
      Count: {count}
      <button onClick={() => inc()}>Increment</button>
      <button onClick={() => dec()}>Decrement</button>
    </div>
  )
}
```

## API

**`createStore(hook) => { ctx, Provider }`**

Creates a context and provider component.

```js
const { ctx, Provider, useProvider } = createStore(useHook)
```

**`CombinedProviders`**

Combined multiple providers.

Prop `providers` is an array.
Each item can be a provider or an object containing `provider` and `args`.
`args` is passed into the first parameter of the hook.

```jsx
<CombinedProviders
  providers={[
    ProviderOne,
    ProviderTwo,
    {
      provider: ProviderThree,
      args: 'initial value',
    },
  ]}
>
  /* ... */
</CombinedProviders>
```

Developed by [Acidic9](https://github.com/Acidic9).
