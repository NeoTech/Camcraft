## #2946 - AGIECUT Classic 2S machine template

Ship a machine library entry for setup. Section 1.1 (p. 4) gives the U and V limits relative to the vertical wire and the rapid speed for each axis. Section 5.2 (p. 44) lists electrode wires. The template includes the listed diameters within the standard Classic range of 0.20 to 0.33 mm. [AGIE technical data C2.1, June 2002](https://static.gindumac.com/products/9ccc9b7e-b589-49db-9843-eff1da09434c/documents/ES-EDM-AGI-2002-00001.pdf)

Keep the X, Y and Z stroke lengths in the description. Centering them on zero would invent the endpoints in the user's setup. The taper rating depends on the guide and work height. Leave the taper limit and guide heights unset until the operator specifies the installation.

The machine entry supplies no post or generator conditions. A wire operation needs a condition from the user's control before CamCraft can generate its path. An operator can save a configured copy of the template without changing the shipped file. Adding a native job writer requires separate control-specific sources and verification.
