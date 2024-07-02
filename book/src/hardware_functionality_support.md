# Hardware & Functionality Support

> RIOT-rs always allows to obtain raw peripherals when using the `riot_rs::spawner` and `riot_rs::task` attributes.
> The table below indicates whether we support using the piece of functionality in a portable manner, trough an abstraction layer and platform-aware configuration.

<!-- We use an HTML table as the GitHub Tables extension does not support cells spanning on multiple rows or columns. -->
<table>
  <thead>
    <tr>
      <th>Chip(s)</th>
      <th>Testing Board</th>
      <!-- The colspan needs to be updated when adding new functionality columns. -->
      <th colspan="6">Functionality</th>
    </tr>
    <tr>
      <th></th>
      <th></th>
      <th>GPIO</th>
      <th>Debug Output</th>
      <th>Logging</th>
      <th>User USB</th>
      <th>Wi-Fi</th>
      <th>Ethernet over USB</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td rowspan=2>RP2040</td>
      <td>Raspberry Pi Pico</td>
      <td class="support-cell">♻️</td>
      <td class="support-cell">✅</td>
      <td class="support-cell">🚧</td>
      <td class="support-cell">✅</td>
      <td class="support-cell">🚫</td>
      <td class="support-cell">✅</td>
    </tr>
    <tr>
      <td>Raspberry Pi Pico W</td>
      <td class="support-cell">♻️</td>
      <td class="support-cell">✅</td>
      <td class="support-cell">🚧</td>
      <td class="support-cell">✅</td>
      <td class="support-cell">✅</td>
      <td class="support-cell">✅</td>
    </tr>
    <tr>
      <td>nRF52xxx</td>
      <td>nRF52840-DK</td>
      <td class="support-cell">♻️</td>
      <td class="support-cell">✅</td>
      <td class="support-cell">🚧</td>
      <td class="support-cell">✅</td>
      <td class="support-cell">🚫</td>
      <td class="support-cell">✅</td>
    </tr>
    <tr>
      <td>nRF5340</td>
      <td>nRF5340-DK</td>
      <td class="support-cell">♻️</td>
      <td class="support-cell">✅</td>
      <td class="support-cell">🚧</td>
      <td class="support-cell">✅</td>
      <td class="support-cell">🚫</td>
      <td class="support-cell">✅</td>
    </tr>
    <tr>
      <td>ESP32-C6</td>
      <td>ESP32-C6-DevKitC-1</td>
      <td class="support-cell">♻️</td>
      <td class="support-cell">🚧</td>
      <td class="support-cell">🚧</td>
      <td class="support-cell">🚧</td>
      <td class="support-cell">🚧</td>
      <td class="support-cell">🚧</td>
    </tr>
  </tbody>
</table>

Key:

✅  supported  
⚠️  supported with some caveats  
❔  needs testing; support may be present but we do not currently test it  
♻️  rework in progress, expect breaking changes  
🚧  work in progress  
❌  not currently supported by RIOT-rs  
🚫  not available on this piece of hardware  

<style>
.support-cell {
  text-align: center;
}
</style>
