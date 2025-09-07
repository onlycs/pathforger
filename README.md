# pathforger (cancelled)

I honestly would have picked a different name but they were all taken by
auto builders (e.g. pathplanner, pathweaver).

## For future me:

- You're not getting PNP results:
    - Check the WebUI
    - Make sure the robot is posting data from `PoseStrategy.MULTI_TAG_PNP_ON_COPROCESSOR`


## Cancellation
Cancelled in 2024 because pathplanner's pathfinding exists

### What this was going to do

- Track enemy robots (locations provided by photon, assignment done by IOU tracker)
- Figure out where they're going and where they're going to be, via a [Kalman Filter](https://wikipedia.org/wiki/Kalman_Filter)
- Generate paths (sets of points, probably) for the robot to follow from a source point to a target point
- Probably more importantly, update that list of points in real-time as enemy robots move

I got around the first two steps done.

And yes, I realize that pathplanner's implementation doesn't do step 4 but, let's face it, all of the math (which I need to understand)
went waaay over my head.
