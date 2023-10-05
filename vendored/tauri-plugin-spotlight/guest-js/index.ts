import { invoke } from '@tauri-apps/api/tauri'

export async function show () {
  void invoke('plugin:spotlight|show')
}

export async function hide () {
  void invoke('plugin:spotlight|hide')
}
