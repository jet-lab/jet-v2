// An effect to pass to atoms so that they utilize browser localStorage
export function localStorageEffect(key: string) {
  return ({ setSelf, onSet }: any) => {
    let savedValue = localStorage.getItem(key);
    if (savedValue != null) {
      if (savedValue[0] === '{' || savedValue[0] === '[') {
        savedValue = JSON.parse(savedValue);
      } else if (savedValue === 'true' || savedValue === 'false') {
        savedValue = JSON.parse(savedValue);
      }
      setSelf(savedValue);
    }

    onSet((newValue: any, _: any, isReset: boolean) => {
      const valueToSet = typeof newValue === 'string' ? newValue : JSON.stringify(newValue);
      isReset ? localStorage.removeItem(key) : localStorage.setItem(key, valueToSet);
    });
  };
}
