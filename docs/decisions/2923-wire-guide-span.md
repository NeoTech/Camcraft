## #2923 - Check the full work height against the wire guides

Check each guide height stated by the operation's machine before slicing the work for verification. Use the same height span as the wire sweep. It includes the programmed planes, the source part and the stock. Add the setup origin's Z coordinate to each stated guide height. The comparison then uses the same guide planes as the post. This also lets a 12 mm workpiece fit at an upper guide height of 2.3 mm and a setup origin of 9.7 mm.

A check on the programmed move alone passed a 12 mm workpiece with its upper guide at 5 mm. A check on the operation planes alone would also miss stock extending above the part. The check applies to vertical cuts and ruled cuts because both pass through the work. A guide may meet the surface. No machine clearance is assumed.

Requiring both guide heights for every vertical cut would prevent use of machines with unstated heights. Enforcing an additional minimum clearance later would require a machine-specific value and a test for its boundary.
