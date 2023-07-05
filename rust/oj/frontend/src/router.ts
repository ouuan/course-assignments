import { createRouter, createWebHashHistory } from 'vue-router';

import HomePage from '~/pages/HomePage.vue';
import ProblemList from '~/pages/ProblemList.vue';
import ProblemSubmission from '~/pages/ProblemSubmission.vue';
import SubmissionListPage from '~/pages/SubmissionListPage.vue';
import SubmissionStatus from '~/pages/SubmissionStatus.vue';
import ContestListPage from '~/pages/ContestListPage.vue';
import ContestPage from '~/pages/ContestPage.vue';
import RankListPage from '~/pages/RankListPage.vue';

const routes = [
  {
    name: '首页',
    path: '/',
    component: HomePage,
  },
  {
    name: '题库',
    path: '/problems',
    component: ProblemList,
  },
  {
    path: '/problem/:id(\\d+)',
    component: ProblemSubmission,
    props: true,
  },
  {
    name: '提交列表',
    path: '/submissions',
    component: SubmissionListPage,
  },
  {
    path: '/submission/:id(\\d+)',
    component: SubmissionStatus,
    props: true,
  },
  {
    name: '比赛列表',
    path: '/contests',
    component: ContestListPage,
  },
  {
    path: '/contest/:id(\\d+)',
    component: ContestPage,
    props: true,
  },
  {
    path: '/contest/:contestId(\\d+)/problem/:id(\\d+)',
    component: ProblemSubmission,
    props: true,
  },
  {
    path: '/contest/:contestId(\\d+)/submissions',
    component: SubmissionListPage,
    props: true,
  },
  {
    name: '排行榜',
    path: '/ranklist',
    component: RankListPage,
  },
];

export default createRouter({
  routes,
  history: createWebHashHistory(),
});
