let client = #{
    global: #{
        set: |key,value| Storage.set(key, value),
        get: |key| Storage_get(key),
        isEmpty: || this.storage.is_empty(),
        clear: |key| this.storage.clear(key),
        clearAll: || this.storage.clear_all(),
    },
};

let console = #{
    log: |value| print(value),
};
